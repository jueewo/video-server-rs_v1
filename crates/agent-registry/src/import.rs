use agent_collection_processor::AgentDefinition;

use crate::models::CreateAgentRequest;

/// Convert a file-based AgentDefinition into a CreateAgentRequest for the registry.
pub fn from_file_definition(
    def: &AgentDefinition,
    workspace_id: Option<&str>,
    file_path: Option<&str>,
) -> CreateAgentRequest {
    let slug = slugify(&def.name);

    CreateAgentRequest {
        slug,
        name: def.name.clone(),
        role: def.role.clone(),
        description: def.description.clone(),
        model: def.model.clone(),
        tools: def.tools.clone(),
        temperature: def.temperature as f64,
        folder_types: def.folder_types.clone(),
        autonomy: def.autonomy.clone(),
        max_iterations: def.max_iterations as i64,
        max_tokens: def.max_tokens as i64,
        timeout: def.timeout as i64,
        max_depth: def.max_depth as i64,
        system_prompt: def.system_prompt.clone(),
        supervisor_id: None,
        can_spawn_sub_agents: false,
        max_sub_agents: 3,
        avatar_url: None,
        color: String::new(),
        tags: Vec::new(),
        source_workspace_id: workspace_id.map(|s| s.to_string()),
        source_file_path: file_path.map(|s| s.to_string()),
    }
}

/// Convert a name to a URL-safe slug (kebab-case).
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
