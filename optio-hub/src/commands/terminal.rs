//! Terminal Tauri Commands
//!
//! Commands for managing interactive remote terminal sessions.

use crate::error::ErrorResponse;
use crate::terminal::TerminalState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Request to start a terminal session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTerminalRequest {
    /// Agent machine ID to connect to
    pub agent_id: String,
    /// Agent's gRPC address (e.g., "http://192.168.1.100:50052")
    pub agent_address: String,
    /// Shell type (optional, defaults to "powershell" on Windows, "bash" on Linux)
    pub shell_type: Option<String>,
    /// Initial terminal width in columns (default 80)
    pub cols: Option<u32>,
    /// Initial terminal height in rows (default 24)
    pub rows: Option<u32>,
}

/// Response for starting a terminal session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTerminalResponse {
    /// Unique session ID for this terminal
    pub session_id: String,
    /// Success indicator
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Request to send input to a terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalInputRequest {
    /// Session ID
    pub session_id: String,
    /// Input data (base64 encoded)
    pub data: String,
}

/// Request to resize a terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalResizeRequest {
    /// Session ID
    pub session_id: String,
    /// New width in columns
    pub cols: u32,
    /// New height in rows
    pub rows: u32,
}

/// Start a new terminal session with an agent
#[tauri::command]
pub async fn start_terminal_session(
    request: StartTerminalRequest,
    terminal_state: State<'_, TerminalState>,
) -> Result<StartTerminalResponse, ErrorResponse> {
    let manager = terminal_state.lock().await;

    match manager
        .start_session(
            request.agent_id,
            request.agent_address,
            request.shell_type,
            request.cols,
            request.rows,
        )
        .await
    {
        Ok(session_id) => Ok(StartTerminalResponse {
            session_id,
            success: true,
            error: None,
        }),
        Err(e) => Ok(StartTerminalResponse {
            session_id: String::new(),
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

/// Send input to a terminal session
#[tauri::command]
pub async fn terminal_send_input(
    request: TerminalInputRequest,
    terminal_state: State<'_, TerminalState>,
) -> Result<bool, ErrorResponse> {
    let manager = terminal_state.lock().await;

    // Decode base64 input
    let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &request.data)
        .map_err(|e| ErrorResponse {
            code: "DECODE_ERROR".to_string(),
            message: format!("Failed to decode input: {}", e),
            details: None,
        })?;

    manager
        .send_input(&request.session_id, data)
        .await
        .map_err(|e| ErrorResponse {
            code: "SEND_ERROR".to_string(),
            message: format!("Failed to send input: {}", e),
            details: None,
        })?;

    Ok(true)
}

/// Resize a terminal session
#[tauri::command]
pub async fn terminal_resize(
    request: TerminalResizeRequest,
    terminal_state: State<'_, TerminalState>,
) -> Result<bool, ErrorResponse> {
    let manager = terminal_state.lock().await;

    manager
        .resize(&request.session_id, request.cols, request.rows)
        .await
        .map_err(|e| ErrorResponse {
            code: "RESIZE_ERROR".to_string(),
            message: format!("Failed to resize terminal: {}", e),
            details: None,
        })?;

    Ok(true)
}

/// Close a terminal session
#[tauri::command]
pub async fn close_terminal_session(
    session_id: String,
    reason: Option<String>,
    terminal_state: State<'_, TerminalState>,
) -> Result<bool, ErrorResponse> {
    let manager = terminal_state.lock().await;

    manager
        .close_session(&session_id, reason)
        .await
        .map_err(|e| ErrorResponse {
            code: "CLOSE_ERROR".to_string(),
            message: format!("Failed to close terminal: {}", e),
            details: None,
        })?;

    Ok(true)
}

/// Check if a terminal session exists
#[tauri::command]
pub async fn terminal_session_exists(
    session_id: String,
    terminal_state: State<'_, TerminalState>,
) -> Result<bool, ErrorResponse> {
    let manager = terminal_state.lock().await;
    Ok(manager.has_session(&session_id).await)
}

/// Get the count of active terminal sessions
#[tauri::command]
pub async fn get_terminal_session_count(
    terminal_state: State<'_, TerminalState>,
) -> Result<usize, ErrorResponse> {
    let manager = terminal_state.lock().await;
    Ok(manager.session_count().await)
}
