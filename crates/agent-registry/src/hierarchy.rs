use std::collections::HashMap;

use crate::models::{AgentTreeNode, RegisteredAgent};

/// Build a forest (list of trees) from a flat list of agents.
/// Root agents have `supervisor_id = None`.
pub fn build_tree(agents: Vec<RegisteredAgent>) -> Vec<AgentTreeNode> {
    // Index agents by id
    let mut by_id: HashMap<i64, RegisteredAgent> = HashMap::new();
    let mut children_map: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut root_ids: Vec<i64> = Vec::new();

    for agent in &agents {
        if let Some(sup_id) = agent.supervisor_id {
            children_map.entry(sup_id).or_default().push(agent.id);
        } else {
            root_ids.push(agent.id);
        }
    }

    for agent in agents {
        by_id.insert(agent.id, agent);
    }

    // Sort roots by name
    root_ids.sort_by(|a, b| {
        let na = by_id.get(a).map(|x| &x.name).unwrap_or(&String::new()).to_lowercase();
        let nb = by_id.get(b).map(|x| &x.name).unwrap_or(&String::new()).to_lowercase();
        na.cmp(&nb)
    });

    root_ids
        .into_iter()
        .filter_map(|id| build_subtree(id, 0, &by_id, &children_map))
        .collect()
}

fn build_subtree(
    id: i64,
    depth: usize,
    by_id: &HashMap<i64, RegisteredAgent>,
    children_map: &HashMap<i64, Vec<i64>>,
) -> Option<AgentTreeNode> {
    let agent = by_id.get(&id)?.clone();
    let child_ids = children_map.get(&id).cloned().unwrap_or_default();
    let children: Vec<AgentTreeNode> = child_ids
        .into_iter()
        .filter_map(|cid| build_subtree(cid, depth + 1, by_id, children_map))
        .collect();

    Some(AgentTreeNode {
        agent,
        children,
        depth,
    })
}

/// Count total descendants in a tree node.
pub fn count_descendants(node: &AgentTreeNode) -> usize {
    node.children.len()
        + node
            .children
            .iter()
            .map(count_descendants)
            .sum::<usize>()
}
