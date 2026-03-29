//! BPMN-to-YAML converter and process simulation support.
//!
//! Converts BPMN 2.0 XML (with optional `agentic:` extension attributes)
//! into the process-engine YAML format for execution.
//!
//! ## Supported BPMN elements
//!
//! | BPMN XML | YAML type |
//! |---|---|
//! | `<bpmn:startEvent>` | `start-event` |
//! | `<bpmn:endEvent>` | `end-event` |
//! | `<bpmn:serviceTask>` | `service-task` |
//! | `<bpmn:userTask>` | `human-task` |
//! | `<bpmn:scriptTask>` | `script-task` |
//! | `<bpmn:task>` with `agentic:taskType="agent"` | `agent-task` |
//! | `<bpmn:task>` (generic) | `service-task` |
//! | `<bpmn:exclusiveGateway>` | `exclusive-gateway` |
//! | `<bpmn:parallelGateway>` | `parallel-gateway` |
//! | `<bpmn:inclusiveGateway>` | `inclusive-gateway` |
//! | `<bpmn:intermediateCatchEvent>` with timer | `timer-event` |

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// Configuration (for workspace processor use)
// ============================================================================

/// Configuration for BPMN simulation (from workspace.yaml)
#[derive(Debug, Clone, Deserialize)]
pub struct BpmnSimulatorConfig {
    pub main_process: String,
    pub config: Option<String>,
    #[serde(default)]
    pub variables: HashMap<String, serde_yaml::Value>,
}

// ============================================================================
// Execution trace types (kept for future simulation overlay)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionTrace {
    pub process_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: ExecutionStatus,
    pub events: Vec<ProcessEvent>,
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Suspended,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessEvent {
    pub timestamp: String,
    pub event_type: String,
    pub element_id: String,
    pub element_name: Option<String>,
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// BPMN-to-YAML converter
// ============================================================================

const BPMN_NS: &str = "http://www.omg.org/spec/BPMN/20100524/MODEL";
const AGENTIC_NS: &str = "http://agentic-bpmn/schema/1.0";

/// Parsed BPMN element.
#[derive(Debug, Clone, Serialize)]
pub struct YamlElement {
    id: String,
    #[serde(rename = "type")]
    element_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<serde_yaml::Value>,
}

/// Parsed sequence flow.
#[derive(Debug, Clone, Serialize)]
pub struct YamlFlow {
    from: String,
    to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
}

/// Process metadata.
#[derive(Debug, Clone, Serialize)]
pub struct YamlProcess {
    id: String,
    name: String,
    version: u32,
}

/// Full YAML output structure.
#[derive(Debug, Clone, Serialize)]
pub struct YamlOutput {
    process: YamlProcess,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    variables: HashMap<String, serde_yaml::Value>,
    elements: Vec<YamlElement>,
    flows: Vec<YamlFlow>,
}

/// Convert BPMN XML string to process-engine YAML string.
pub fn bpmn_to_yaml(bpmn_xml: &str) -> Result<String> {
    let output = bpmn_to_yaml_struct(bpmn_xml)?;
    serde_yaml::to_string(&output).map_err(|e| anyhow!("YAML serialization failed: {e}"))
}

/// Convert BPMN XML to structured output (useful for API responses).
pub fn bpmn_to_yaml_struct(bpmn_xml: &str) -> Result<YamlOutput> {
    let doc = roxmltree::Document::parse(bpmn_xml)
        .map_err(|e| anyhow!("Invalid BPMN XML: {e}"))?;

    // Find the first <process> element
    let process_node = doc
        .descendants()
        .find(|n| n.has_tag_name((BPMN_NS, "process")))
        .or_else(|| {
            // Fallback: look for process without namespace
            doc.descendants().find(|n| n.tag_name().name() == "process")
        })
        .ok_or_else(|| anyhow!("No <process> element found in BPMN XML"))?;

    let process_id = process_node.attribute("id").unwrap_or("process_1").to_string();
    let process_name = process_node
        .attribute("name")
        .unwrap_or(&process_id)
        .to_string();

    let mut elements = Vec::new();
    let mut flows = Vec::new();

    // Track default sequence flow for exclusive gateways
    let mut gateway_defaults: HashMap<String, String> = HashMap::new();

    // First pass: collect gateway default flows
    for node in process_node.children().filter(|n| n.is_element()) {
        if let Some(default_flow) = node.attribute("default") {
            if let Some(id) = node.attribute("id") {
                gateway_defaults.insert(id.to_string(), default_flow.to_string());
            }
        }
    }

    // Second pass: convert elements
    for node in process_node.children().filter(|n| n.is_element()) {
        let local_name = node.tag_name().name();
        let id = match node.attribute("id") {
            Some(id) => id.to_string(),
            None => continue,
        };
        let name = node.attribute("name").map(|s| s.to_string());

        match local_name {
            "startEvent" => {
                elements.push(YamlElement {
                    id,
                    element_type: "start-event".to_string(),
                    name,
                    config: None,
                });
            }
            "endEvent" => {
                elements.push(YamlElement {
                    id,
                    element_type: "end-event".to_string(),
                    name,
                    config: None,
                });
            }
            "task" => {
                // Check for agentic task type
                let agentic_type = node
                    .attribute((AGENTIC_NS, "taskType"))
                    .or_else(|| node.attribute("agentic:taskType"));

                if agentic_type == Some("agent") {
                    elements.push(build_agent_element(&node, id, name));
                } else {
                    elements.push(YamlElement {
                        id,
                        element_type: "service-task".to_string(),
                        name,
                        config: None,
                    });
                }
            }
            "serviceTask" => {
                // Check if it's also an agentic task
                let agentic_type = node
                    .attribute((AGENTIC_NS, "taskType"))
                    .or_else(|| node.attribute("agentic:taskType"));

                if agentic_type == Some("agent") {
                    elements.push(build_agent_element(&node, id, name));
                } else {
                    elements.push(YamlElement {
                        id,
                        element_type: "service-task".to_string(),
                        name,
                        config: None,
                    });
                }
            }
            "userTask" => {
                let config = build_human_task_config(&node);
                elements.push(YamlElement {
                    id,
                    element_type: "human-task".to_string(),
                    name,
                    config,
                });
            }
            "scriptTask" => {
                let config = build_script_task_config(&node);
                elements.push(YamlElement {
                    id,
                    element_type: "script-task".to_string(),
                    name,
                    config,
                });
            }
            "exclusiveGateway" => {
                let config = build_gateway_config(&node);
                elements.push(YamlElement {
                    id,
                    element_type: "exclusive-gateway".to_string(),
                    name,
                    config,
                });
            }
            "parallelGateway" => {
                let config = build_gateway_config(&node);
                elements.push(YamlElement {
                    id,
                    element_type: "parallel-gateway".to_string(),
                    name,
                    config,
                });
            }
            "inclusiveGateway" => {
                let config = build_gateway_config(&node);
                elements.push(YamlElement {
                    id,
                    element_type: "inclusive-gateway".to_string(),
                    name,
                    config,
                });
            }
            "intermediateCatchEvent" => {
                // Check for timer definition
                let has_timer = node.children().any(|c| {
                    c.tag_name().name() == "timerEventDefinition"
                });
                if has_timer {
                    elements.push(YamlElement {
                        id,
                        element_type: "timer-event".to_string(),
                        name,
                        config: None,
                    });
                }
            }
            "sequenceFlow" => {
                let source = node.attribute("sourceRef").unwrap_or("").to_string();
                let target = node.attribute("targetRef").unwrap_or("").to_string();
                if source.is_empty() || target.is_empty() {
                    continue;
                }

                // Extract condition from conditionExpression child
                let condition = node
                    .children()
                    .find(|c| c.tag_name().name() == "conditionExpression")
                    .and_then(|c| c.text())
                    .map(|t| t.trim().to_string())
                    .filter(|s| !s.is_empty());

                // Check if this is a default flow for any gateway
                let is_default = gateway_defaults
                    .values()
                    .any(|default_id| default_id == &id);

                flows.push(YamlFlow {
                    from: source,
                    to: target,
                    condition,
                    default: if is_default { Some(true) } else { None },
                });
            }
            _ => {
                // Skip unknown elements (di:*, bpmndi:*, etc.)
            }
        }
    }

    Ok(YamlOutput {
        process: YamlProcess {
            id: process_id,
            name: process_name,
            version: 1,
        },
        variables: HashMap::new(),
        elements,
        flows,
    })
}

/// Load a BPMN file and convert to YAML.
pub fn load_and_convert(file_path: &Path) -> Result<String> {
    let xml = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read BPMN file: {e}"))?;
    bpmn_to_yaml(&xml)
}

// ============================================================================
// Element builders
// ============================================================================

fn build_agent_element(node: &roxmltree::Node, id: String, name: Option<String>) -> YamlElement {
    let mut config = serde_yaml::Mapping::new();

    if let Some(agent_id) = get_agentic_attr(node, "agentId") {
        config.insert(
            serde_yaml::Value::String("agent".to_string()),
            serde_yaml::Value::String(agent_id),
        );
    }

    if let Some(model) = get_agentic_attr(node, "model") {
        config.insert(
            serde_yaml::Value::String("model".to_string()),
            serde_yaml::Value::String(model),
        );
    }

    if let Some(mode) = get_agentic_attr(node, "reflectionMode") {
        config.insert(
            serde_yaml::Value::String("reflection_mode".to_string()),
            serde_yaml::Value::String(mode),
        );
    }

    if let Some(confidence) = get_agentic_attr(node, "confidence") {
        if let Ok(n) = confidence.parse::<u64>() {
            config.insert(
                serde_yaml::Value::String("confidence".to_string()),
                serde_yaml::Value::Number(n.into()),
            );
        }
    }

    if let Some(max_iter) = get_agentic_attr(node, "maxIterations") {
        if let Ok(n) = max_iter.parse::<u64>() {
            config.insert(
                serde_yaml::Value::String("max_iterations".to_string()),
                serde_yaml::Value::Number(n.into()),
            );
        }
    }

    // Default output variable based on element ID
    config.insert(
        serde_yaml::Value::String("output_var".to_string()),
        serde_yaml::Value::String(format!("{}_result", sanitize_id(&id))),
    );

    YamlElement {
        id,
        element_type: "agent-task".to_string(),
        name,
        config: Some(serde_yaml::Value::Mapping(config)),
    }
}

fn build_human_task_config(node: &roxmltree::Node) -> Option<serde_yaml::Value> {
    let mut config = serde_yaml::Mapping::new();

    // Extract assignee from BPMN performer/potentialOwner if available
    if let Some(assignee) = node.attribute("assignee") {
        config.insert(
            serde_yaml::Value::String("assignee".to_string()),
            serde_yaml::Value::String(assignee.to_string()),
        );
    }

    if config.is_empty() {
        None
    } else {
        Some(serde_yaml::Value::Mapping(config))
    }
}

fn build_script_task_config(node: &roxmltree::Node) -> Option<serde_yaml::Value> {
    // Look for <script> child element content
    let script_text = node
        .children()
        .find(|c| c.tag_name().name() == "script")
        .and_then(|c| c.text())
        .map(|t| t.trim().to_string());

    if let Some(expr) = script_text {
        let mut config = serde_yaml::Mapping::new();
        config.insert(
            serde_yaml::Value::String("expression".to_string()),
            serde_yaml::Value::String(expr),
        );
        Some(serde_yaml::Value::Mapping(config))
    } else {
        None
    }
}

fn build_gateway_config(node: &roxmltree::Node) -> Option<serde_yaml::Value> {
    let mut config = serde_yaml::Mapping::new();

    if let Some(collab) = get_agentic_attr(node, "collaborationMode") {
        config.insert(
            serde_yaml::Value::String("collaboration_mode".to_string()),
            serde_yaml::Value::String(collab),
        );
    }

    if let Some(merge) = get_agentic_attr(node, "mergingStrategy") {
        config.insert(
            serde_yaml::Value::String("merging_strategy".to_string()),
            serde_yaml::Value::String(merge),
        );
    }

    if config.is_empty() {
        None
    } else {
        Some(serde_yaml::Value::Mapping(config))
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Get an agentic namespace attribute from a node.
/// Tries both namespaced and prefixed forms.
fn get_agentic_attr(node: &roxmltree::Node, local_name: &str) -> Option<String> {
    node.attribute((AGENTIC_NS, local_name))
        .or_else(|| {
            // Fallback: look for agentic:name as a plain attribute
            let prefixed = format!("agentic:{local_name}");
            node.attributes()
                .find(|a| a.name() == prefixed)
                .map(|a| a.value())
        })
        .map(|s| s.to_string())
}

/// Sanitize an element ID for use as a variable name.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let yaml = r#"
main_process: order-flow.bpmn
variables:
  timeout: 3600
  retries: 3
"#;
        let config: BpmnSimulatorConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.main_process, "order-flow.bpmn");
        assert_eq!(config.variables.len(), 2);
    }

    #[test]
    fn convert_simple_bpmn() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
             xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <process id="simple_process" name="Simple Process">
    <startEvent id="start" name="Start" />
    <task id="task1" name="Do Something" />
    <endEvent id="end" name="End" />
    <sequenceFlow id="f1" sourceRef="start" targetRef="task1" />
    <sequenceFlow id="f2" sourceRef="task1" targetRef="end" />
  </process>
</definitions>"#;

        let output = bpmn_to_yaml_struct(xml).unwrap();
        assert_eq!(output.process.id, "simple_process");
        assert_eq!(output.elements.len(), 3);
        assert_eq!(output.flows.len(), 2);
        assert_eq!(output.elements[0].element_type, "start-event");
        assert_eq!(output.elements[1].element_type, "service-task"); // generic task
        assert_eq!(output.elements[2].element_type, "end-event");
    }

    #[test]
    fn convert_agent_task() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
             xmlns:agentic="http://agentic-bpmn/schema/1.0">
  <process id="agent_process" name="Agent Process">
    <startEvent id="start" />
    <task id="review" name="Code Review"
          agentic:taskType="agent"
          agentic:agentId="code-reviewer"
          agentic:model="claude-sonnet"
          agentic:reflectionMode="self"
          agentic:confidence="80"
          agentic:maxIterations="5" />
    <endEvent id="end" />
    <sequenceFlow id="f1" sourceRef="start" targetRef="review" />
    <sequenceFlow id="f2" sourceRef="review" targetRef="end" />
  </process>
</definitions>"#;

        let output = bpmn_to_yaml_struct(xml).unwrap();
        let agent_elem = &output.elements[1];
        assert_eq!(agent_elem.element_type, "agent-task");
        assert_eq!(agent_elem.name.as_deref(), Some("Code Review"));

        let config = agent_elem.config.as_ref().unwrap();
        let map = config.as_mapping().unwrap();
        assert_eq!(
            map.get(&serde_yaml::Value::String("agent".into())),
            Some(&serde_yaml::Value::String("code-reviewer".into()))
        );
        assert_eq!(
            map.get(&serde_yaml::Value::String("reflection_mode".into())),
            Some(&serde_yaml::Value::String("self".into()))
        );
        assert_eq!(
            map.get(&serde_yaml::Value::String("confidence".into())),
            Some(&serde_yaml::Value::Number(80.into()))
        );
    }

    #[test]
    fn convert_gateways_and_conditions() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <process id="gw_process" name="Gateway Process">
    <startEvent id="start" />
    <exclusiveGateway id="gw1" name="Check Amount" default="f_default" />
    <task id="approve" name="Auto Approve" />
    <task id="review" name="Manual Review" />
    <endEvent id="end" />
    <sequenceFlow id="f1" sourceRef="start" targetRef="gw1" />
    <sequenceFlow id="f_low" sourceRef="gw1" targetRef="approve">
      <conditionExpression>${amount} &lt; 1000</conditionExpression>
    </sequenceFlow>
    <sequenceFlow id="f_default" sourceRef="gw1" targetRef="review" />
    <sequenceFlow id="f3" sourceRef="approve" targetRef="end" />
    <sequenceFlow id="f4" sourceRef="review" targetRef="end" />
  </process>
</definitions>"#;

        let output = bpmn_to_yaml_struct(xml).unwrap();

        // Check gateway
        let gw = output.elements.iter().find(|e| e.id == "gw1").unwrap();
        assert_eq!(gw.element_type, "exclusive-gateway");

        // Check conditional flow
        let cond_flow = output.flows.iter().find(|f| f.from == "gw1" && f.to == "approve").unwrap();
        assert!(cond_flow.condition.is_some());
        assert!(cond_flow.condition.as_ref().unwrap().contains("amount"));

        // Check default flow
        let default_flow = output.flows.iter().find(|f| f.from == "gw1" && f.to == "review").unwrap();
        assert_eq!(default_flow.default, Some(true));
    }

    #[test]
    fn convert_typed_tasks() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <process id="typed" name="Typed Tasks">
    <startEvent id="start" />
    <serviceTask id="svc1" name="Call API" />
    <userTask id="human1" name="Approve" />
    <scriptTask id="script1" name="Set Flag">
      <script>approved = true</script>
    </scriptTask>
    <endEvent id="end" />
  </process>
</definitions>"#;

        let output = bpmn_to_yaml_struct(xml).unwrap();
        assert_eq!(output.elements[1].element_type, "service-task");
        assert_eq!(output.elements[2].element_type, "human-task");
        assert_eq!(output.elements[3].element_type, "script-task");

        // Check script config
        let script_config = output.elements[3].config.as_ref().unwrap();
        let map = script_config.as_mapping().unwrap();
        assert_eq!(
            map.get(&serde_yaml::Value::String("expression".into())),
            Some(&serde_yaml::Value::String("approved = true".into()))
        );
    }

    #[test]
    fn convert_agentic_gateway() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
             xmlns:agentic="http://agentic-bpmn/schema/1.0">
  <process id="collab" name="Collaboration">
    <parallelGateway id="fork" agentic:collaborationMode="role" />
    <parallelGateway id="join" agentic:mergingStrategy="leader-driven" />
  </process>
</definitions>"#;

        let output = bpmn_to_yaml_struct(xml).unwrap();
        let fork = &output.elements[0];
        let join = &output.elements[1];

        let fork_config = fork.config.as_ref().unwrap().as_mapping().unwrap();
        assert_eq!(
            fork_config.get(&serde_yaml::Value::String("collaboration_mode".into())),
            Some(&serde_yaml::Value::String("role".into()))
        );

        let join_config = join.config.as_ref().unwrap().as_mapping().unwrap();
        assert_eq!(
            join_config.get(&serde_yaml::Value::String("merging_strategy".into())),
            Some(&serde_yaml::Value::String("leader-driven".into()))
        );
    }

    #[test]
    fn yaml_roundtrip() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <process id="test" name="Test">
    <startEvent id="start" />
    <endEvent id="end" />
    <sequenceFlow id="f1" sourceRef="start" targetRef="end" />
  </process>
</definitions>"#;

        let yaml_str = bpmn_to_yaml(xml).unwrap();
        assert!(yaml_str.contains("start-event"));
        assert!(yaml_str.contains("end-event"));

        // Verify it's valid YAML
        let _: serde_yaml::Value = serde_yaml::from_str(&yaml_str).unwrap();
    }
}
