use serde::{Deserialize, Serialize};

/// A registered agent stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredAgent {
    pub id: i64,
    pub slug: String,
    pub user_id: String,
    pub name: String,
    pub role: String,
    pub description: String,
    pub model: String,
    pub tools: Vec<String>,
    pub temperature: f64,
    pub folder_types: Vec<String>,
    pub autonomy: String,
    pub max_iterations: i64,
    pub max_tokens: i64,
    pub timeout: i64,
    pub max_depth: i64,
    pub system_prompt: String,
    // Hierarchy
    pub supervisor_id: Option<i64>,
    pub can_spawn_sub_agents: bool,
    pub max_sub_agents: i64,
    // Appearance
    pub avatar_url: Option<String>,
    pub color: String,
    // Tags
    pub tags: Vec<String>,
    // Provenance
    pub source_workspace_id: Option<String>,
    pub source_file_path: Option<String>,
    // Status
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create or update an agent.
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub slug: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default)]
    pub folder_types: Vec<String>,
    #[serde(default = "default_autonomy")]
    pub autonomy: String,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: i64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i64,
    #[serde(default = "default_timeout")]
    pub timeout: i64,
    #[serde(default = "default_max_depth")]
    pub max_depth: i64,
    #[serde(default)]
    pub system_prompt: String,
    pub supervisor_id: Option<i64>,
    #[serde(default)]
    pub can_spawn_sub_agents: bool,
    #[serde(default = "default_max_sub_agents")]
    pub max_sub_agents: i64,
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub source_workspace_id: Option<String>,
    pub source_file_path: Option<String>,
}

/// Partial update request — all fields optional.
#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub model: Option<String>,
    pub tools: Option<Vec<String>>,
    pub temperature: Option<f64>,
    pub folder_types: Option<Vec<String>>,
    pub autonomy: Option<String>,
    pub max_iterations: Option<i64>,
    pub max_tokens: Option<i64>,
    pub timeout: Option<i64>,
    pub max_depth: Option<i64>,
    pub system_prompt: Option<String>,
    pub supervisor_id: Option<Option<i64>>,
    pub can_spawn_sub_agents: Option<bool>,
    pub max_sub_agents: Option<i64>,
    pub avatar_url: Option<Option<String>>,
    pub color: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

/// Per-workspace overrides for an assigned agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomy: Option<String>,
}

/// Request to set a supervisor.
#[derive(Debug, Deserialize)]
pub struct SetSupervisorRequest {
    pub supervisor_id: i64,
}

/// Request to assign an agent to a workspace.
#[derive(Debug, Deserialize)]
pub struct AssignAgentRequest {
    #[serde(default)]
    pub overrides: AgentOverrides,
}

/// Tree node for hierarchy visualization.
#[derive(Debug, Clone, Serialize)]
pub struct AgentTreeNode {
    pub agent: RegisteredAgent,
    pub children: Vec<AgentTreeNode>,
    pub depth: usize,
}

fn default_model() -> String { String::new() }
fn default_temperature() -> f64 { 1.0 }
fn default_autonomy() -> String { "supervised".to_string() }
fn default_max_iterations() -> i64 { 10 }
fn default_max_tokens() -> i64 { 4096 }
fn default_timeout() -> i64 { 300 }
fn default_max_depth() -> i64 { 3 }
fn default_max_sub_agents() -> i64 { 3 }
