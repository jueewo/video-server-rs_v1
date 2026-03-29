//! Process YAML definition parser.
//!
//! Parses a human-writable YAML DSL into an in-memory process graph.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// YAML structures (deserialized directly)
// ============================================================================

/// Top-level process YAML document.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessYaml {
    pub process: ProcessMeta,
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub flows: Vec<SequenceFlow>,
}

/// Process metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessMeta {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: i64,
}

/// A single process element (task, event, gateway).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Element {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
}

/// A sequence flow connecting two elements.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SequenceFlow {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub default: bool,
}

fn default_version() -> i64 {
    1
}

// ============================================================================
// Parsed process graph (built from YAML)
// ============================================================================

/// An in-memory process definition ready for execution.
#[derive(Debug, Clone)]
pub struct ProcessGraph {
    pub meta: ProcessMeta,
    pub initial_variables: HashMap<String, serde_json::Value>,
    pub elements: HashMap<String, Element>,
    /// Outgoing flows per element: element_id → [(target_id, condition, is_default)]
    pub outgoing: HashMap<String, Vec<OutgoingFlow>>,
    /// Incoming flow count per element (for parallel join detection).
    pub incoming_count: HashMap<String, usize>,
    /// The start-event element ID.
    pub start_element: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OutgoingFlow {
    pub target: String,
    pub condition: Option<String>,
    pub is_default: bool,
}

// ============================================================================
// Parsing
// ============================================================================

/// Parse a YAML string into a ProcessGraph.
pub fn parse_process_yaml(yaml: &str) -> Result<ProcessGraph, ProcessParseError> {
    let doc: ProcessYaml =
        serde_yaml::from_str(yaml).map_err(|e| ProcessParseError::Yaml(e.to_string()))?;

    let mut elements = HashMap::new();
    let mut start_element = None;

    for el in &doc.elements {
        if elements.contains_key(&el.id) {
            return Err(ProcessParseError::DuplicateElement(el.id.clone()));
        }
        if el.element_type == "start-event" {
            if start_element.is_some() {
                return Err(ProcessParseError::MultipleStartEvents);
            }
            start_element = Some(el.id.clone());
        }
        elements.insert(el.id.clone(), el.clone());
    }

    if start_element.is_none() {
        return Err(ProcessParseError::NoStartEvent);
    }

    // Build adjacency lists
    let mut outgoing: HashMap<String, Vec<OutgoingFlow>> = HashMap::new();
    let mut incoming_count: HashMap<String, usize> = HashMap::new();

    for flow in &doc.flows {
        if !elements.contains_key(&flow.from) {
            return Err(ProcessParseError::UnknownElement(flow.from.clone()));
        }
        if !elements.contains_key(&flow.to) {
            return Err(ProcessParseError::UnknownElement(flow.to.clone()));
        }

        outgoing
            .entry(flow.from.clone())
            .or_default()
            .push(OutgoingFlow {
                target: flow.to.clone(),
                condition: flow.condition.clone(),
                is_default: flow.default,
            });

        *incoming_count.entry(flow.to.clone()).or_insert(0) += 1;
    }

    Ok(ProcessGraph {
        meta: doc.process,
        initial_variables: doc.variables,
        elements,
        outgoing,
        incoming_count,
        start_element,
    })
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ProcessParseError {
    #[error("YAML parse error: {0}")]
    Yaml(String),
    #[error("duplicate element ID: {0}")]
    DuplicateElement(String),
    #[error("no start-event defined")]
    NoStartEvent,
    #[error("multiple start-event elements")]
    MultipleStartEvents,
    #[error("flow references unknown element: {0}")]
    UnknownElement(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = r#"
process:
  id: test-process
  name: Test Process

variables:
  count: 0

elements:
  - id: start
    type: start-event
  - id: do-work
    type: script-task
    name: Do Work
    config:
      expression: "count = 1"
  - id: end
    type: end-event

flows:
  - from: start
    to: do-work
  - from: do-work
    to: end
"#;

    #[test]
    fn parse_simple_process() {
        let graph = parse_process_yaml(SAMPLE_YAML).unwrap();
        assert_eq!(graph.meta.id, "test-process");
        assert_eq!(graph.elements.len(), 3);
        assert_eq!(graph.start_element, Some("start".to_string()));
        assert_eq!(graph.outgoing["start"].len(), 1);
        assert_eq!(graph.outgoing["start"][0].target, "do-work");
        assert_eq!(*graph.incoming_count.get("end").unwrap(), 1);
    }

    #[test]
    fn reject_no_start_event() {
        let yaml = r#"
process:
  id: broken
  name: Broken
elements:
  - id: end
    type: end-event
flows: []
"#;
        let err = parse_process_yaml(yaml).unwrap_err();
        assert!(matches!(err, ProcessParseError::NoStartEvent));
    }

    #[test]
    fn reject_duplicate_ids() {
        let yaml = r#"
process:
  id: dup
  name: Dup
elements:
  - id: start
    type: start-event
  - id: start
    type: end-event
flows: []
"#;
        let err = parse_process_yaml(yaml).unwrap_err();
        assert!(matches!(err, ProcessParseError::DuplicateElement(_)));
    }
}
