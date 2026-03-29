//! Process runtime engine.
//!
//! BPMN-inspired process execution with YAML definitions, a token-based state
//! machine, and pluggable task executors (service, agent, human, script).

pub mod agent;
pub mod agent_memory;
pub mod definition;
pub mod engine;
pub mod executor;
pub mod routes;
pub mod scheduler;
pub mod service;
pub mod variables;

use std::sync::Arc;

/// Shared state for process engine API routes.
#[derive(Clone)]
pub struct ProcessEngineState {
    pub engine: Arc<engine::ProcessEngine>,
}
