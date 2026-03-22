use sqlx::SqlitePool;

use crate::models::{AgentOverrides, CreateAgentRequest, RegisteredAgent, UpdateAgentRequest};

/// Row type matching the agent_definitions table.
#[derive(sqlx::FromRow)]
struct AgentRow {
    id: i64,
    slug: String,
    user_id: String,
    name: String,
    role: String,
    description: String,
    model: String,
    tools: String,        // JSON array
    temperature: f64,
    folder_types: String, // JSON array
    autonomy: String,
    max_iterations: i64,
    max_tokens: i64,
    timeout: i64,
    max_depth: i64,
    system_prompt: String,
    supervisor_id: Option<i64>,
    can_spawn_sub_agents: i64, // SQLite boolean
    max_sub_agents: i64,
    avatar_url: Option<String>,
    color: String,
    tags: String,           // JSON array
    source_workspace_id: Option<String>,
    source_file_path: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

impl From<AgentRow> for RegisteredAgent {
    fn from(r: AgentRow) -> Self {
        Self {
            id: r.id,
            slug: r.slug,
            user_id: r.user_id,
            name: r.name,
            role: r.role,
            description: r.description,
            model: r.model,
            tools: serde_json::from_str(&r.tools).unwrap_or_default(),
            temperature: r.temperature,
            folder_types: serde_json::from_str(&r.folder_types).unwrap_or_default(),
            autonomy: r.autonomy,
            max_iterations: r.max_iterations,
            max_tokens: r.max_tokens,
            timeout: r.timeout,
            max_depth: r.max_depth,
            system_prompt: r.system_prompt,
            supervisor_id: r.supervisor_id,
            can_spawn_sub_agents: r.can_spawn_sub_agents != 0,
            max_sub_agents: r.max_sub_agents,
            avatar_url: r.avatar_url,
            color: r.color,
            tags: serde_json::from_str(&r.tags).unwrap_or_default(),
            source_workspace_id: r.source_workspace_id,
            source_file_path: r.source_file_path,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

const SELECT_COLS: &str = "id, slug, user_id, name, role, description, model, tools, \
    temperature, folder_types, autonomy, max_iterations, max_tokens, timeout, max_depth, \
    system_prompt, supervisor_id, can_spawn_sub_agents, max_sub_agents, avatar_url, \
    color, tags, source_workspace_id, source_file_path, status, created_at, updated_at";

pub async fn insert_agent(
    pool: &SqlitePool,
    user_id: &str,
    req: &CreateAgentRequest,
) -> Result<i64, sqlx::Error> {
    let tools_json = serde_json::to_string(&req.tools).unwrap_or_else(|_| "[]".into());
    let folder_types_json =
        serde_json::to_string(&req.folder_types).unwrap_or_else(|_| "[]".into());
    let can_spawn = if req.can_spawn_sub_agents { 1i64 } else { 0 };

    let tags_json = serde_json::to_string(&req.tags).unwrap_or_else(|_| "[]".into());

    let result = sqlx::query(
        "INSERT INTO agent_definitions \
         (slug, user_id, name, role, description, model, tools, temperature, folder_types, \
          autonomy, max_iterations, max_tokens, timeout, max_depth, system_prompt, \
          supervisor_id, can_spawn_sub_agents, max_sub_agents, avatar_url, color, tags, \
          source_workspace_id, source_file_path) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&req.slug)
    .bind(user_id)
    .bind(&req.name)
    .bind(&req.role)
    .bind(&req.description)
    .bind(&req.model)
    .bind(&tools_json)
    .bind(req.temperature)
    .bind(&folder_types_json)
    .bind(&req.autonomy)
    .bind(req.max_iterations)
    .bind(req.max_tokens)
    .bind(req.timeout)
    .bind(req.max_depth)
    .bind(&req.system_prompt)
    .bind(req.supervisor_id)
    .bind(can_spawn)
    .bind(req.max_sub_agents)
    .bind(&req.avatar_url)
    .bind(&req.color)
    .bind(&tags_json)
    .bind(&req.source_workspace_id)
    .bind(&req.source_file_path)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_agent(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<RegisteredAgent>, sqlx::Error> {
    let row: Option<AgentRow> = sqlx::query_as(&format!(
        "SELECT {SELECT_COLS} FROM agent_definitions WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn get_agent_by_slug(
    pool: &SqlitePool,
    user_id: &str,
    slug: &str,
) -> Result<Option<RegisteredAgent>, sqlx::Error> {
    let row: Option<AgentRow> = sqlx::query_as(&format!(
        "SELECT {SELECT_COLS} FROM agent_definitions WHERE user_id = ? AND slug = ?"
    ))
    .bind(user_id)
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn list_user_agents(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<RegisteredAgent>, sqlx::Error> {
    let rows: Vec<AgentRow> = sqlx::query_as(&format!(
        "SELECT {SELECT_COLS} FROM agent_definitions WHERE user_id = ? ORDER BY name"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn update_agent(
    pool: &SqlitePool,
    id: i64,
    user_id: &str,
    req: &UpdateAgentRequest,
) -> Result<bool, sqlx::Error> {
    // Fetch current and apply updates, then write back
    let current = match get_agent(pool, id).await? {
        Some(a) if a.user_id == user_id => a,
        _ => return Ok(false),
    };

    let name = req.name.as_deref().unwrap_or(&current.name);
    let role = req.role.as_deref().unwrap_or(&current.role);
    let description = req.description.as_deref().unwrap_or(&current.description);
    let model = req.model.as_deref().unwrap_or(&current.model);
    let tools = req.tools.as_ref().unwrap_or(&current.tools);
    let tools_json = serde_json::to_string(tools).unwrap_or_else(|_| "[]".into());
    let temperature = req.temperature.unwrap_or(current.temperature);
    let folder_types = req.folder_types.as_ref().unwrap_or(&current.folder_types);
    let folder_types_json = serde_json::to_string(folder_types).unwrap_or_else(|_| "[]".into());
    let autonomy = req.autonomy.as_deref().unwrap_or(&current.autonomy);
    let max_iterations = req.max_iterations.unwrap_or(current.max_iterations);
    let max_tokens = req.max_tokens.unwrap_or(current.max_tokens);
    let timeout = req.timeout.unwrap_or(current.timeout);
    let max_depth = req.max_depth.unwrap_or(current.max_depth);
    let system_prompt = req.system_prompt.as_deref().unwrap_or(&current.system_prompt);
    let supervisor_id = match &req.supervisor_id {
        Some(v) => *v,
        None => current.supervisor_id,
    };
    let can_spawn = if req.can_spawn_sub_agents.unwrap_or(current.can_spawn_sub_agents) { 1i64 } else { 0 };
    let max_sub_agents = req.max_sub_agents.unwrap_or(current.max_sub_agents);
    let avatar_url = match &req.avatar_url {
        Some(v) => v.clone(),
        None => current.avatar_url.clone(),
    };
    let color = req.color.as_deref().unwrap_or(&current.color);
    let tags = req.tags.as_ref().unwrap_or(&current.tags);
    let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".into());
    let status = req.status.as_deref().unwrap_or(&current.status);

    let result = sqlx::query(
        "UPDATE agent_definitions SET \
         name = ?, role = ?, description = ?, model = ?, tools = ?, temperature = ?, \
         folder_types = ?, autonomy = ?, max_iterations = ?, max_tokens = ?, timeout = ?, \
         max_depth = ?, system_prompt = ?, supervisor_id = ?, can_spawn_sub_agents = ?, \
         max_sub_agents = ?, avatar_url = ?, color = ?, tags = ?, status = ?, \
         updated_at = datetime('now') \
         WHERE id = ? AND user_id = ?",
    )
    .bind(name)
    .bind(role)
    .bind(description)
    .bind(model)
    .bind(&tools_json)
    .bind(temperature)
    .bind(&folder_types_json)
    .bind(autonomy)
    .bind(max_iterations)
    .bind(max_tokens)
    .bind(timeout)
    .bind(max_depth)
    .bind(system_prompt)
    .bind(supervisor_id)
    .bind(can_spawn)
    .bind(max_sub_agents)
    .bind(&avatar_url)
    .bind(color)
    .bind(&tags_json)
    .bind(status)
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_agent(
    pool: &SqlitePool,
    id: i64,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM agent_definitions WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn set_supervisor(
    pool: &SqlitePool,
    id: i64,
    supervisor_id: Option<i64>,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE agent_definitions SET supervisor_id = ?, updated_at = datetime('now') \
         WHERE id = ? AND user_id = ?",
    )
    .bind(supervisor_id)
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_subordinates(
    pool: &SqlitePool,
    supervisor_id: i64,
) -> Result<Vec<RegisteredAgent>, sqlx::Error> {
    let rows: Vec<AgentRow> = sqlx::query_as(&format!(
        "SELECT {SELECT_COLS} FROM agent_definitions WHERE supervisor_id = ? ORDER BY name"
    ))
    .bind(supervisor_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

// -- Workspace agent assignments --

pub async fn assign_to_workspace(
    pool: &SqlitePool,
    workspace_id: &str,
    agent_id: i64,
    overrides: &AgentOverrides,
) -> Result<(), sqlx::Error> {
    let overrides_json = serde_json::to_string(overrides).unwrap_or_else(|_| "{}".into());

    sqlx::query(
        "INSERT INTO workspace_agents (workspace_id, agent_id, overrides) \
         VALUES (?, ?, ?) \
         ON CONFLICT(workspace_id, agent_id) DO UPDATE SET overrides = excluded.overrides",
    )
    .bind(workspace_id)
    .bind(agent_id)
    .bind(&overrides_json)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_from_workspace(
    pool: &SqlitePool,
    workspace_id: &str,
    agent_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM workspace_agents WHERE workspace_id = ? AND agent_id = ?",
    )
    .bind(workspace_id)
    .bind(agent_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Workspace agent row with overrides JSON.
#[derive(sqlx::FromRow)]
struct WorkspaceAgentRow {
    // agent_definitions columns
    id: i64,
    slug: String,
    user_id: String,
    name: String,
    role: String,
    description: String,
    model: String,
    tools: String,
    temperature: f64,
    folder_types: String,
    autonomy: String,
    max_iterations: i64,
    max_tokens: i64,
    timeout: i64,
    max_depth: i64,
    system_prompt: String,
    supervisor_id: Option<i64>,
    can_spawn_sub_agents: i64,
    max_sub_agents: i64,
    avatar_url: Option<String>,
    color: String,
    tags: String,
    source_workspace_id: Option<String>,
    source_file_path: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
    // workspace_agents column
    overrides: String,
}

pub async fn get_workspace_agents(
    pool: &SqlitePool,
    workspace_id: &str,
) -> Result<Vec<(RegisteredAgent, AgentOverrides)>, sqlx::Error> {
    let rows: Vec<WorkspaceAgentRow> = sqlx::query_as(&format!(
        "SELECT a.{}, wa.overrides \
         FROM workspace_agents wa \
         JOIN agent_definitions a ON a.id = wa.agent_id \
         WHERE wa.workspace_id = ? \
         ORDER BY a.name",
        SELECT_COLS.replace("id,", "a.id,")
    ))
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let overrides: AgentOverrides =
                serde_json::from_str(&r.overrides).unwrap_or_default();
            let agent = AgentRow {
                id: r.id,
                slug: r.slug,
                user_id: r.user_id,
                name: r.name,
                role: r.role,
                description: r.description,
                model: r.model,
                tools: r.tools,
                temperature: r.temperature,
                folder_types: r.folder_types,
                autonomy: r.autonomy,
                max_iterations: r.max_iterations,
                max_tokens: r.max_tokens,
                timeout: r.timeout,
                max_depth: r.max_depth,
                system_prompt: r.system_prompt,
                supervisor_id: r.supervisor_id,
                can_spawn_sub_agents: r.can_spawn_sub_agents,
                max_sub_agents: r.max_sub_agents,
                avatar_url: r.avatar_url,
                color: r.color,
                tags: r.tags,
                source_workspace_id: r.source_workspace_id,
                source_file_path: r.source_file_path,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            (agent.into(), overrides)
        })
        .collect())
}

/// Upsert: insert or update if slug already exists for user.
pub async fn upsert_agent(
    pool: &SqlitePool,
    user_id: &str,
    req: &CreateAgentRequest,
) -> Result<i64, sqlx::Error> {
    // Check if exists
    if let Some(existing) = get_agent_by_slug(pool, user_id, &req.slug).await? {
        let update = UpdateAgentRequest {
            name: Some(req.name.clone()),
            role: Some(req.role.clone()),
            description: Some(req.description.clone()),
            model: Some(req.model.clone()),
            tools: Some(req.tools.clone()),
            temperature: Some(req.temperature),
            folder_types: Some(req.folder_types.clone()),
            autonomy: Some(req.autonomy.clone()),
            max_iterations: Some(req.max_iterations),
            max_tokens: Some(req.max_tokens),
            timeout: Some(req.timeout),
            max_depth: Some(req.max_depth),
            system_prompt: Some(req.system_prompt.clone()),
            supervisor_id: None,
            can_spawn_sub_agents: Some(req.can_spawn_sub_agents),
            max_sub_agents: Some(req.max_sub_agents),
            avatar_url: None,
            color: Some(req.color.clone()),
            tags: Some(req.tags.clone()),
            status: None,
        };
        update_agent(pool, existing.id, user_id, &update).await?;
        Ok(existing.id)
    } else {
        insert_agent(pool, user_id, req).await
    }
}

/// Check if setting supervisor_id would create a cycle.
pub async fn would_create_cycle(
    pool: &SqlitePool,
    agent_id: i64,
    proposed_supervisor_id: i64,
) -> Result<bool, sqlx::Error> {
    if agent_id == proposed_supervisor_id {
        return Ok(true);
    }

    // Walk up the ancestor chain from proposed_supervisor_id
    let mut current = Some(proposed_supervisor_id);
    let mut depth = 0;
    while let Some(cid) = current {
        if depth > 50 {
            return Ok(true); // safety limit
        }
        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT supervisor_id FROM agent_definitions WHERE id = ?",
        )
        .bind(cid)
        .fetch_optional(pool)
        .await?;

        match row {
            Some((Some(parent_id),)) => {
                if parent_id == agent_id {
                    return Ok(true); // cycle detected
                }
                current = Some(parent_id);
            }
            _ => break,
        }
        depth += 1;
    }

    Ok(false)
}
