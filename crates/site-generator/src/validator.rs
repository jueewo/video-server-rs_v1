/// Page element validator.
///
/// Validates compiled page.json elements against the element schema registry.
/// Reports errors (missing required fields) and warnings (unknown fields)
/// during site generation so authors catch problems before the Astro build.

use std::path::Path;

use anyhow::Result;
use tracing::{error, warn};

use crate::element_schemas::{find_schema, FieldType};

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn print(&self, source: &str) {
        for e in &self.errors {
            error!("[validator] {} — ERROR: {}", source, e);
        }
        for w in &self.warnings {
            warn!("[validator] {} — WARN: {}", source, w);
        }
    }
}

/// Validate a compiled `page.json` file.
/// Returns a `ValidationReport` with errors/warnings; never fails the build.
pub fn validate_page_json(path: &Path) -> Result<ValidationReport> {
    let text = std::fs::read_to_string(path)?;
    let page: serde_json::Value = serde_json::from_str(&text)?;

    let mut report = ValidationReport::default();

    let elements = match page.get("elements") {
        Some(serde_json::Value::Array(arr)) => arr,
        _ => {
            report.errors.push("page.json has no top-level 'elements' array".to_string());
            return Ok(report);
        }
    };

    for (i, element) in elements.iter().enumerate() {
        let element_type = element
            .get("element")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");

        let context = format!("element[{}] ({})", i, element_type);

        // Skip draft elements
        if element.get("draft").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }

        // Validate Section children recursively
        if element_type == "Section" {
            if let Some(serde_json::Value::Array(children)) = element.get("elements") {
                for (j, child) in children.iter().enumerate() {
                    validate_element(child, &format!("{}.elements[{}]", context, j), &mut report);
                }
            }
        }

        validate_element(element, &context, &mut report);
    }

    Ok(report)
}

fn validate_element(
    element: &serde_json::Value,
    context: &str,
    report: &mut ValidationReport,
) {
    let element_type = element
        .get("element")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");

    let schema = match find_schema(element_type) {
        Some(s) => s,
        None => {
            report.warnings.push(format!(
                "{}: unknown element type '{}'",
                context, element_type
            ));
            return;
        }
    };

    // Collect known top-level JSON keys for this element
    let element_obj = match element.as_object() {
        Some(o) => o,
        None => return,
    };

    // Check required fields — support both flat and legacy props/content nesting
    for field in schema.required_fields() {
        let present = element_obj.contains_key(field.name)
            || element_obj
                .get("props")
                .and_then(|p| p.as_object())
                .map_or(false, |p| p.contains_key(field.name))
            || element_obj
                .get("content")
                .and_then(|c| c.as_object())
                .map_or(false, |c| c.contains_key(field.name));

        if !present {
            report.errors.push(format!(
                "{}: required field '{}' is missing",
                context, field.name
            ));
        }
    }

    // Type-check flat fields (best-effort)
    for field in schema.fields {
        let value = element_obj.get(field.name);
        if let Some(v) = value {
            if let Some(msg) = check_field_type(field.name, v, field.field_type) {
                report.warnings.push(format!("{}: {}", context, msg));
            }
        }
    }

    // Warn about unknown flat keys (skip structural/meta keys)
    let known_names: std::collections::HashSet<&str> =
        schema.fields.iter().map(|f| f.name).collect();
    let meta_keys = ["element", "draft", "anim", "props", "content"];
    for key in element_obj.keys() {
        if !known_names.contains(key.as_str()) && !meta_keys.contains(&key.as_str()) {
            report.warnings.push(format!(
                "{}: unknown field '{}'",
                context, key
            ));
        }
    }
}

fn check_field_type(name: &str, value: &serde_json::Value, expected: FieldType) -> Option<String> {
    let ok = match expected {
        FieldType::String => value.is_string(),
        FieldType::Bool => value.is_boolean(),
        FieldType::Number => value.is_number(),
        FieldType::StringArray => {
            value.is_array()
                && value
                    .as_array()
                    .map_or(true, |arr| arr.iter().all(|v| v.is_string()))
        }
        FieldType::Array => value.is_array(),
        FieldType::Object => value.is_object(),
        FieldType::Any => true,
    };

    if ok {
        None
    } else {
        Some(format!(
            "field '{}' has unexpected type (got {})",
            name,
            type_name(value)
        ))
    }
}

fn type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
