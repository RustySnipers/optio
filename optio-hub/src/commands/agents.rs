//! Agent management Tauri commands
//!
//! Commands for managing and querying connected optio-agent instances.

use crate::db::{
    ActionLog, ActionLogRepository, ActionStatus, ActionType, Agent, AgentRepository,
    AgentStatus, Database, TelemetryRecord, TelemetryRepository, TelemetryStats,
};
use crate::error::ErrorResponse;
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

// ============================================================================
// Telemetry History Commands
// ============================================================================

/// Get telemetry history for an agent
#[tauri::command]
pub async fn get_agent_history(
    agent_id: String,
    limit: Option<u32>,
    db: State<'_, Database>,
) -> Result<Vec<TelemetryRecord>, ErrorResponse> {
    let repo = TelemetryRepository::new(&db);
    repo.get_history(&agent_id, limit.unwrap_or(100)).map_err(Into::into)
}

/// Get telemetry statistics for an agent
#[tauri::command]
pub async fn get_agent_telemetry_stats(
    agent_id: String,
    hours: Option<i64>,
    db: State<'_, Database>,
) -> Result<TelemetryStats, ErrorResponse> {
    let repo = TelemetryRepository::new(&db);
    repo.get_stats(&agent_id, hours.unwrap_or(24)).map_err(Into::into)
}

/// Cleanup old telemetry records
#[tauri::command]
pub async fn cleanup_telemetry(
    days_to_keep: Option<i64>,
    db: State<'_, Database>,
) -> Result<usize, ErrorResponse> {
    let repo = TelemetryRepository::new(&db);
    repo.cleanup_old_records(days_to_keep.unwrap_or(30)).map_err(Into::into)
}

// ============================================================================
// Audit Log Commands
// ============================================================================

/// Request to send a command to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCommandRequest {
    /// Target agent machine ID
    pub agent_id: String,
    /// Command type (execute_script, etc.)
    pub command_type: String,
    /// Command payload (JSON string)
    pub payload_json: String,
    /// Priority (higher = more urgent)
    pub priority: Option<u32>,
    /// Timeout in seconds
    pub timeout_seconds: Option<u32>,
}

/// Response from sending a command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCommandResponse {
    /// Unique command ID
    pub command_id: String,
    /// Whether the command was queued successfully
    pub success: bool,
    /// Message
    pub message: String,
}

/// Send a command to an agent (queues it for the next heartbeat)
#[tauri::command]
pub async fn send_agent_command(
    request: SendCommandRequest,
    db: State<'_, Database>,
) -> Result<SendCommandResponse, ErrorResponse> {
    let conn = db.conn.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".to_string(),
        message: format!("Failed to lock database: {}", e),
        details: None,
    })?;

    let command_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert the command into the pending queue
    conn.execute(
        r#"
        INSERT INTO pending_commands (command_id, machine_id, command_type, payload_json, priority, timeout_seconds, status, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7)
        "#,
        rusqlite::params![
            command_id,
            request.agent_id,
            request.command_type,
            request.payload_json,
            request.priority.unwrap_or(0) as i32,
            request.timeout_seconds.unwrap_or(300) as i32,
            now,
        ],
    ).map_err(|e| ErrorResponse {
        code: "INSERT_ERROR".to_string(),
        message: format!("Failed to queue command: {}", e),
        details: None,
    })?;

    // Create an action log entry
    let log_repo = ActionLogRepository::new(&db);
    let action_log = ActionLog::new(
        request.agent_id.clone(),
        ActionType::ScriptExec,
        Some(sanitize_command_content(&request.payload_json)),
        true, // user initiated
    )
    .with_metadata(serde_json::json!({
        "command_id": command_id,
        "command_type": request.command_type,
    }));

    if let Err(e) = log_repo.insert(&action_log) {
        tracing::warn!("Failed to insert action log: {}", e);
    }

    drop(conn); // Release lock before logging

    tracing::info!(
        command_id = %command_id,
        agent_id = %request.agent_id,
        command_type = %request.command_type,
        "Command queued for agent"
    );

    Ok(SendCommandResponse {
        command_id,
        success: true,
        message: "Command queued successfully".to_string(),
    })
}

/// Get audit logs with optional filtering
#[tauri::command]
pub async fn get_audit_logs(
    agent_id: Option<String>,
    limit: Option<u32>,
    db: State<'_, Database>,
) -> Result<Vec<ActionLog>, ErrorResponse> {
    let repo = ActionLogRepository::new(&db);
    repo.list(agent_id.as_deref(), limit.unwrap_or(100))
        .map_err(Into::into)
}

/// Get audit logs for a specific agent
#[tauri::command]
pub async fn get_agent_audit_logs(
    agent_id: String,
    limit: Option<u32>,
    db: State<'_, Database>,
) -> Result<Vec<ActionLog>, ErrorResponse> {
    let repo = ActionLogRepository::new(&db);
    repo.get_by_agent(&agent_id, limit.unwrap_or(100))
        .map_err(Into::into)
}

/// Get audit logs by action type
#[tauri::command]
pub async fn get_audit_logs_by_type(
    action_type: String,
    limit: Option<u32>,
    db: State<'_, Database>,
) -> Result<Vec<ActionLog>, ErrorResponse> {
    let repo = ActionLogRepository::new(&db);
    let at = ActionType::from(action_type.as_str());
    repo.get_by_type(at, limit.unwrap_or(100))
        .map_err(Into::into)
}

/// Update the status of an audit log entry
#[tauri::command]
pub async fn update_audit_log_status(
    log_id: String,
    status: String,
    db: State<'_, Database>,
) -> Result<bool, ErrorResponse> {
    let repo = ActionLogRepository::new(&db);
    let action_status = ActionStatus::from(status.as_str());
    repo.update_status(&log_id, action_status)
        .map_err(Into::into)
}

/// Cleanup old audit logs
#[tauri::command]
pub async fn cleanup_audit_logs(
    days_to_keep: Option<i64>,
    db: State<'_, Database>,
) -> Result<usize, ErrorResponse> {
    let repo = ActionLogRepository::new(&db);
    repo.cleanup_old_logs(days_to_keep.unwrap_or(90))
        .map_err(Into::into)
}

/// Log a terminal session start
pub fn log_terminal_session_start(
    db: &Database,
    agent_id: &str,
    session_id: &str,
) -> Result<(), crate::error::OptioError> {
    let repo = ActionLogRepository::new(db);
    let log = ActionLog::new(
        agent_id.to_string(),
        ActionType::TerminalSession,
        Some("Terminal session started".to_string()),
        true,
    )
    .with_session(session_id.to_string())
    .with_status(ActionStatus::InProgress);

    repo.insert(&log)
}

/// Log a terminal session end
pub fn log_terminal_session_end(
    db: &Database,
    agent_id: &str,
    session_id: &str,
    success: bool,
) -> Result<(), crate::error::OptioError> {
    let repo = ActionLogRepository::new(db);
    let log = ActionLog::new(
        agent_id.to_string(),
        ActionType::TerminalSession,
        Some("Terminal session ended".to_string()),
        true,
    )
    .with_session(session_id.to_string())
    .with_status(if success {
        ActionStatus::Success
    } else {
        ActionStatus::Failure
    });

    repo.insert(&log)
}

/// Sanitize command content for logging (remove sensitive data)
fn sanitize_command_content(content: &str) -> String {
    // For now, just truncate very long content
    if content.len() > 1000 {
        format!("{}... [truncated]", &content[..1000])
    } else {
        content.to_string()
    }
}
