//! Agent management Tauri commands
//!
//! Commands for managing and querying connected optio-agent instances.

use crate::db::{Agent, AgentRepository, AgentStatus, Database};
use crate::error::{ErrorResponse, OptioResult};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Summary statistics for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStats {
    pub total: i64,
    pub online: i64,
    pub offline: i64,
    pub unknown: i64,
}

/// List all registered agents
#[tauri::command]
pub async fn get_agents(db: State<'_, Database>) -> Result<Vec<Agent>, ErrorResponse> {
    let repo = AgentRepository::new(&db);
    repo.list().map_err(Into::into)
}

/// Get a specific agent by machine_id
#[tauri::command]
pub async fn get_agent(
    machine_id: String,
    db: State<'_, Database>,
) -> Result<Option<Agent>, ErrorResponse> {
    let repo = AgentRepository::new(&db);
    repo.get(&machine_id).map_err(Into::into)
}

/// List agents filtered by status
#[tauri::command]
pub async fn get_agents_by_status(
    status: String,
    db: State<'_, Database>,
) -> Result<Vec<Agent>, ErrorResponse> {
    let repo = AgentRepository::new(&db);
    let agent_status = AgentStatus::from(status.as_str());
    repo.list_by_status(agent_status).map_err(Into::into)
}

/// Get agent statistics summary
#[tauri::command]
pub async fn get_agent_stats(db: State<'_, Database>) -> Result<AgentStats, ErrorResponse> {
    let repo = AgentRepository::new(&db);
    let counts = repo.count_by_status()?;

    Ok(AgentStats {
        total: counts.values().sum(),
        online: *counts.get("online").unwrap_or(&0),
        offline: *counts.get("offline").unwrap_or(&0),
        unknown: *counts.get("unknown").unwrap_or(&0) + *counts.get("error").unwrap_or(&0),
    })
}

/// Delete an agent from the database
#[tauri::command]
pub async fn delete_agent(
    machine_id: String,
    db: State<'_, Database>,
) -> Result<bool, ErrorResponse> {
    let repo = AgentRepository::new(&db);
    repo.delete(&machine_id).map_err(Into::into)
}

/// Mark stale agents as offline
/// This should be called periodically (e.g., every minute)
#[tauri::command]
pub async fn refresh_agent_status(db: State<'_, Database>) -> Result<usize, ErrorResponse> {
    let repo = AgentRepository::new(&db);
    repo.mark_stale_agents_offline().map_err(Into::into)
}
