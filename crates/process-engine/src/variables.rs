//! Variable resolution and condition evaluation.
//!
//! Supports `${var}` and `${var.field}` substitution in strings,
//! and simple comparison conditions like `"${x} < 50"`.

use serde_json::Value;

/// Resolve all `${...}` references in a template string against the variables.
pub fn resolve_variables(template: &str, variables: &Value) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                var_name.push(c);
            }
            let value = resolve_path(&var_name, variables);
            result.push_str(&value_to_string(&value));
        } else {
            result.push(c);
        }
    }

    result
}

/// Evaluate a condition string (after variable resolution).
///
/// Supports: `<`, `>`, `==`, `!=`, `<=`, `>=` on numbers, strings, and booleans.
/// Returns `true` if the condition holds, `false` otherwise.
pub fn evaluate_condition(condition: &str, variables: &Value) -> bool {
    let resolved = resolve_variables(condition, variables);
    let resolved = resolved.trim();

    // Try each operator (longest first to avoid partial matches)
    for op in &["<=", ">=", "!=", "==", "<", ">"] {
        if let Some(pos) = resolved.find(op) {
            let left = resolved[..pos].trim();
            let right = resolved[pos + op.len()..].trim();
            return compare(left, right, op);
        }
    }

    // No operator found — treat as boolean
    resolved == "true"
}

/// Resolve a dotted path like "result.score" against a JSON value.
fn resolve_path(path: &str, variables: &Value) -> Value {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = variables;

    for part in parts {
        match current {
            Value::Object(map) => {
                if let Some(v) = map.get(part) {
                    current = v;
                } else {
                    return Value::Null;
                }
            }
            _ => return Value::Null,
        }
    }

    current.clone()
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn compare(left: &str, right: &str, op: &str) -> bool {
    // Try numeric comparison first
    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
        return match op {
            "<" => l < r,
            ">" => l > r,
            "<=" => l <= r,
            ">=" => l >= r,
            "==" => (l - r).abs() < f64::EPSILON,
            "!=" => (l - r).abs() >= f64::EPSILON,
            _ => false,
        };
    }

    // Boolean comparison
    if let (Some(l), Some(r)) = (parse_bool(left), parse_bool(right)) {
        return match op {
            "==" => l == r,
            "!=" => l != r,
            _ => false,
        };
    }

    // String comparison
    match op {
        "==" => left == right,
        "!=" => left != right,
        "<" => left < right,
        ">" => left > right,
        "<=" => left <= right,
        ">=" => left >= right,
        _ => false,
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolve_simple_variable() {
        let vars = json!({"name": "Alice", "count": 42});
        assert_eq!(resolve_variables("Hello ${name}!", &vars), "Hello Alice!");
        assert_eq!(resolve_variables("Count: ${count}", &vars), "Count: 42");
    }

    #[test]
    fn resolve_nested_path() {
        let vars = json!({"result": {"score": 85, "label": "pass"}});
        assert_eq!(resolve_variables("${result.score}", &vars), "85");
        assert_eq!(resolve_variables("${result.label}", &vars), "pass");
    }

    #[test]
    fn resolve_missing_variable() {
        let vars = json!({"x": 1});
        assert_eq!(resolve_variables("${missing}", &vars), "");
    }

    #[test]
    fn evaluate_numeric_conditions() {
        let vars = json!({"score": 75});
        assert!(evaluate_condition("${score} > 50", &vars));
        assert!(!evaluate_condition("${score} < 50", &vars));
        assert!(evaluate_condition("${score} == 75", &vars));
        assert!(evaluate_condition("${score} <= 75", &vars));
        assert!(evaluate_condition("${score} >= 75", &vars));
        assert!(evaluate_condition("${score} != 100", &vars));
    }

    #[test]
    fn evaluate_boolean_conditions() {
        let vars = json!({"approved": true});
        assert!(evaluate_condition("${approved} == true", &vars));
        assert!(!evaluate_condition("${approved} == false", &vars));
    }

    #[test]
    fn evaluate_string_conditions() {
        let vars = json!({"status": "active"});
        assert!(evaluate_condition("${status} == active", &vars));
        assert!(!evaluate_condition("${status} == inactive", &vars));
    }

    #[test]
    fn bare_boolean_expression() {
        let vars = json!({"flag": true});
        assert!(evaluate_condition("${flag}", &vars));
    }
}
